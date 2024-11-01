import UniformTypeIdentifiers

public extension UTType {
    static var wrappType: UTType {
        UTType.init(exportedAs: "io.github.webrogue-runtime.wrapp")
    }
}
