
#include <emscripten/emscripten.h>
#include <emscripten/bind.h>
#include <emscripten/val.h>
#include <utility>
#include <valarray>
#include <optional>
// #include <iostream>
#include <cstring>
#include <sstream>
#include <mutex>
#include <libxml/parser.h>
#include <libxml/xmlIO.h>
#include <libxml/xmlschemas.h>
#include <lz4.h>

class U8Reader {
private:
    std::valarray<uint8_t> _buffer;
public:
    uint32_t _pos;
    U8Reader(uint8_t* ptr, uint64_t len) : _buffer(ptr, len) {
        this->_pos = 0;
    }

    [[nodiscard]] const char* ptr() const {
        return (const char*)&this->_buffer[this->_pos];
    }

    uint8_t readU8() {
        auto result = this->_buffer[this->_pos];
        this->_pos += 1;
        return result;
    }

    uint32_t readU32() {
        auto b4 = this->readU8();
        auto b3 = this->readU8();
        auto b2 = this->readU8();
        auto b1 = this->readU8();
        return b1 | (b2 << 8) | (b3 << 16) | (b4 << 24);
    }

    uint32_t readU32_LE() {
        auto b1 = this->readU8();
        auto b2 = this->readU8();
        auto b3 = this->readU8();
        auto b4 = this->readU8();
        return b1 | (b2 << 8) | (b3 << 16) | (b4 << 24);
    }

    uint64_t readU64() {
        uint64_t b2 = this->readU32();
        uint32_t b1 = this->readU32();
        return b1
               | b2 << 32;
    }

    std::string readString() {
        auto s_l = this->readU32();
        auto s = std::string((const char*) this->ptr(), (int) s_l);
        this->_pos += s_l;
        // std::cout << "string len:" << s_l << " got len: " << s.length() << std::endl;
        return s;
    }
};

const uint32_t MAGIC_BYTE = 0x58534442;

class SchemaBundle {
private:
    std::string _name;
    std::string _target_ns;
    std::string _schema_location;
    xmlSchemaPtr _ptr;
public:
    SchemaBundle(
            std::string name,
            std::string target_ns,
            std::string schema_location,
            xmlSchemaPtr ptr
    ) :
            _name(name),
            _target_ns(target_ns),
            _schema_location(schema_location),
            _ptr(ptr) {

    }

    xmlSchemaPtr ptr() const {
        return this->_ptr;
    }

    void destroy() {
        xmlSchemaFree(this->_ptr);
    }
};

class Range {
public:
    uint64_t start;
    uint64_t end;

    Range(
            uint64_t start_,
            uint64_t end_
    ) :
            start(start_),
            end(end_)
    {

    }
};

class ReadCtx {
public:
    const char* ptr;
    int offset;
    int size;
    ReadCtx(const char* ptr_, int size_);
};
class SchemaBundleReader: public U8Reader {
private:
    std::string _name;
    std::string _target_ns;
    std::string _schema_location;
    std::map<std::string, Range> _map;
    Range _entrypoint;
    Range _schemas;
public:
    std::optional<std::string> error;
    SchemaBundleReader(uint8_t* ptr, uint64_t len);
    bool contains(const char* filename);
    ReadCtx* newContext(const char* filename);
    SchemaBundle read();
};

ReadCtx::ReadCtx(const char* ptr_, int size_) : ptr(ptr_), offset(0), size(size_) {

}

class LibXml2Instance
{
public:
    LibXml2Instance(LibXml2Instance const&) = delete;
    LibXml2Instance& operator=(LibXml2Instance const&) = delete;

    static std::shared_ptr<LibXml2Instance> instance()
    {
        static std::shared_ptr<LibXml2Instance> s{new LibXml2Instance};
        return s;
    }

    static int matchRuntimeFn(char const *filename) {
        if (LibXml2Instance::instance()->reader->contains(filename)) {
            // std::cout << "found file: '" << filename << "'" << std::endl;
            return 1;
        }
        return 0;
    }

    static void* openRuntimeFn(char const *filename) {
        return LibXml2Instance::instance()->reader->newContext(filename);
    }

    static int readRuntimeFn(void *context, char* buffer, int len) {
        auto l = len;
        auto ctx = (ReadCtx*)context;
        auto ptr = ctx->ptr + ctx->offset;
        if (l > ctx->size) {
            l = ctx->size;
        }
        std::memcpy(buffer, ptr, l);
        ctx->size -= l;
        ctx->offset += l;
        return l;
    }

    static int closeRuntimeFn(void *context) {
        auto ctx = (ReadCtx*)context;
        delete ctx;
        return 0;
    }
    SchemaBundle read(const SchemaBundleReader& reader_) {
        this->reader = reader_;
        return this->reader->read();
    }
private:
    std::optional<SchemaBundleReader> reader;
    LibXml2Instance() {
        xmlInitParser();
        xmlRegisterInputCallbacks(
            LibXml2Instance::matchRuntimeFn,
            LibXml2Instance::openRuntimeFn,
            LibXml2Instance::readRuntimeFn,
            LibXml2Instance::closeRuntimeFn
        );
    }
};
SchemaBundleReader::SchemaBundleReader(uint8_t* ptr, uint64_t len)
        :
        U8Reader(ptr, len),
        _schemas(Range(0,0)),
        _entrypoint(Range(0,0))
{
    if (this->readU32() != MAGIC_BYTE) {
        this->error = std::optional(std::string ("invalid file format"));
        return;
    }
    auto head_size = this->readU64();
    std::stringstream ss;
    this->_name = this->readString();
    this->_target_ns = this->readString();
    ss << this->_target_ns << " " << this->_name;
    this->_schema_location = ss.str();
    // std::cout << "current schema name: '" << this->_name << "' current schema target_ns '" << this->_target_ns << "' pos: " << this->_pos << std::endl;
    while (true) {
        if (this->_pos == head_size) {
            break;
        }
        if (this->_pos > head_size) {
            this->error = std::optional(std::string ("invalid file header"));
            return;
        }
        // std::cout << "read entrypoint" << std::endl;
        auto is_entrypoint = this->readU8();
        // std::cout << "is entrypoint ? " << is_entrypoint << std::endl;
        // std::cout << "read start" << std::endl;
        auto start = this->readU64();
        // std::cout << "start: " << start << std::endl;
        // std::cout << "read end" << std::endl;
        auto end = this->readU64();
        // std::cout << "end: " << end << std::endl;
        if (is_entrypoint == 1) {
            this->_entrypoint = Range(start + head_size, end + head_size);
        }
        // std::cout << "read name" << std::endl;
        auto name = this->readString();
        // std::cout << "name: " << name << std::endl;
        this->_map.insert(std::pair{name, Range(start + head_size, end + head_size)});
    }
    this->_schemas.start = head_size;
    this->_schemas.end = len;
}

bool SchemaBundleReader::contains(const char* filename) {
    return this->_map.find(filename) != this->_map.end();
}

SchemaBundle SchemaBundleReader::read() {
    // std::cout << "get xml schema ptr via libxml2: " << this->_name << std::endl;
    this->_pos = this->_entrypoint.start;
    auto len = this->_entrypoint.end - this->_entrypoint.start;
    // std::cout << std::string(this->ptr(), len) << std::endl;
    xmlSchemaParserCtxtPtr parser = xmlSchemaNewMemParserCtxt(this->ptr(), len);
    xmlSchemaPtr ptr = xmlSchemaParse(parser);
    return SchemaBundle {
        this->_name,
        this->_target_ns,
        this->_schema_location,
        ptr
    };
}

ReadCtx *SchemaBundleReader::newContext(const char *filename) {
    auto entry = this->_map.at(filename);
    this->_pos = entry.start;
    auto len = entry.end - entry.start;
    return new ReadCtx(this->ptr(), (int) len);
}

class XmlValidatorError {
private:
    uint8_t level;
    uint32_t line;
    std::string message;
public:
    XmlValidatorError(uint8_t level, uint32_t line, std::string message)
            : level(level), line(line), message(std::move(message))
    {
    }

    [[nodiscard]] uint8_t getLevel() const {
        return this->level;
    }

    [[nodiscard]] uint32_t getLine() const {
        return this->line;
    }

    [[nodiscard]] std::string getMessage() const {
        return this->message;
    }
};


class XmlValidatonContext {
public:
    std::vector<XmlValidatorError> errors;
    void append(uint8_t level, uint32_t line, std::string message) {
        this->errors.emplace_back( level, line, std::move(message) );
    }
    static void error_callback(void *userData, const xmlError *error) {
        auto ctx = (XmlValidatonContext*)userData;
        ctx->append(error->level, error->line, std::string(error->message));
    }
};

class XmlValidator {
private:
    std::mutex mtx;
    char* _buffer;
    int _buffer_len;
    std::optional<SchemaBundle> _schema_bundle;
    std::shared_ptr<LibXml2Instance> _instance;
public:
    explicit XmlValidator(const emscripten::val &uint8ArrayObject) {
        this->_instance = LibXml2Instance::instance();
        auto length = uint8ArrayObject["length"].as<unsigned int>();
        std::vector<char> buffer;
        buffer.resize(length);
        auto memory = emscripten::val::module_property("HEAPU8")["buffer"];
        auto memoryView = uint8ArrayObject["constructor"].new_(memory, reinterpret_cast<uintptr_t>(buffer.data()), length);
        memoryView.call<void>("set", uint8ArrayObject);
        U8Reader rdr((uint8_t*)buffer.data(), length);
        int decompressed_len = (int) rdr.readU32_LE();
        char* regen_buffer = (char*)malloc(decompressed_len);
        this->_buffer_len = LZ4_decompress_safe((char*)buffer.data() + 4, regen_buffer, (int)length - 4, decompressed_len);
        this->_buffer = regen_buffer;
    }

    ~XmlValidator() {
        this->_schema_bundle->destroy();
    }

    void init(emscripten::val on_error_cb) {
        SchemaBundleReader rdr((uint8_t*)this->_buffer, this->_buffer_len);
        if (rdr.error.has_value()) {
            on_error_cb(rdr.error.value());
        }
        mtx.lock();
        this->_schema_bundle = this->_instance->read(rdr);
        free(this->_buffer);
        mtx.unlock();
    }

    int validateXml(std::string xml, emscripten::val callback) {
        XmlValidatonContext validationContext;

        auto input = xmlParserInputBufferCreateMem(xml.c_str(), xml.size(), xmlCharEncoding::XML_CHAR_ENCODING_UTF8);
        xmlSchemaValidCtxtPtr ctx = xmlSchemaNewValidCtxt(this->_schema_bundle->ptr());
        xmlSchemaSetValidStructuredErrors(
            ctx,
            XmlValidatonContext::error_callback,
            &validationContext
        );
        auto result = xmlSchemaValidateStream(
                ctx,
                input,
                xmlCharEncoding::XML_CHAR_ENCODING_UTF8,
                nullptr,
                nullptr
        );
        xmlSchemaFreeValidCtxt(ctx);
        for (auto & element : validationContext.errors) {
            callback(element);
        }
        return result;
    }
};

EMSCRIPTEN_BINDINGS(xml_validator) {
    emscripten::class_<XmlValidator>("XmlValidator")
            .constructor<const emscripten::val&>()
            .function("init", &XmlValidator::init)
            .function("validateXml", &XmlValidator::validateXml)
            ;
    emscripten::class_<XmlValidatorError>("XmlValidatorError")
        .constructor<uint8_t, uint32_t, std::string>()
        .property("level", &XmlValidatorError::getLevel)
        .property("line", &XmlValidatorError::getLine)
        .property("message", &XmlValidatorError::getMessage)
        ;
}
