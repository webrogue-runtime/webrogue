import Foundation
import Combine

public final class WrappStorage: ObservableObject {
    let wrappDirectoryPath: String
    let fileManager: FileManager

    @Published public var refs: [WrappRef]

    public init() {
        let fileManager = FileManager.default
        let documentDirectoryPath = fileManager.urls(
            for: .documentDirectory,
            in: .userDomainMask
        ).first!.relativePath
        let wrappDirectoryPath = documentDirectoryPath + "/.wrapp"
        if !fileManager.fileExists(atPath: wrappDirectoryPath) {
            try! fileManager.createDirectory(atPath: wrappDirectoryPath, withIntermediateDirectories: true)
        }
        self.wrappDirectoryPath = wrappDirectoryPath
        self.fileManager = fileManager
        self.refs = []

        updateRefs()
    }

    private func updateRefs() {
        refs.removeAll()
        let fileNames = try! fileManager.contentsOfDirectory(atPath: wrappDirectoryPath)
        for fileName in fileNames {
            let filePath = wrappDirectoryPath + "/" + fileName
            guard
                filePath.hasSuffix(".wrapp"),
                let fileHandle = FileHandle(forReadingAtPath: filePath),
                let metadata = WrappMetadata(fileHandle: fileHandle)
            else { continue }
            refs.append(WrappRef(path: filePath, metadata: metadata))
        }
    }

    @discardableResult
    public func store(_ url: URL) -> WrappRef? {
        guard
            let fileHandle = FileHandle(forReadingAtPath: url.relativePath),
            let metadata = WrappMetadata(fileHandle: fileHandle)
        else {
            return nil
        }
        let newPath = wrappDirectoryPath + "/" + metadata.sha256 + ".wrapp"
        do {
            try fileManager.copyItem(atPath: url.relativePath, toPath: newPath)
        } catch {
            return nil
        }

        updateRefs()
        return refs.first { $0.metadata.sha256 == metadata.sha256 }
     }
}
