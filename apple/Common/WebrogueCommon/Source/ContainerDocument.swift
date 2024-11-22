import UniformTypeIdentifiers
import SwiftUI

public struct ContainerDocument: FileDocument {
    public static var readableContentTypes: [UTType] { [.webc] }

    public init(configuration: ReadConfiguration) throws {
        // todo somehow check configuration.file without loading whole file to memory
    }

    public func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        throw CocoaError(.featureUnsupported)
    }
}
