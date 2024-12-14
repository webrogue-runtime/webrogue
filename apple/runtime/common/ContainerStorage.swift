import Foundation
import Combine

public final class ContainerStorage: ObservableObject {
    let storageDirectoryPath: String
    let fileManager: FileManager

    @Published public var refs: [ContainerReference]

    public init() {
        let fileManager = FileManager.default
        let documentDirectoryPath = fileManager.urls(
            for: .documentDirectory,
            in: .userDomainMask
        ).first!.relativePath
        let containersDirectoryPath = documentDirectoryPath + "/.webrogue_containers"
        if !fileManager.fileExists(atPath: containersDirectoryPath) {
            try! fileManager.createDirectory(atPath: containersDirectoryPath, withIntermediateDirectories: true)
        }
        self.storageDirectoryPath = containersDirectoryPath
        self.fileManager = fileManager
        self.refs = []

        updateRefs()
    }

    private func updateRefs() {
        refs.removeAll()
        let fileNames = try! fileManager.contentsOfDirectory(atPath: storageDirectoryPath)
        for fileName in fileNames {
            let filePath = storageDirectoryPath + "/" + fileName
            guard
                filePath.hasSuffix(".webc"),
                let fileHandle = FileHandle(forReadingAtPath: filePath),
                let metadata = ContainerMetadata(fileHandle: fileHandle)
            else { continue }
            refs.append(ContainerReference(path: filePath, metadata: metadata))
        }
    }

    @discardableResult
    public func store(_ url: URL) -> ContainerReference? {
        guard
            let fileHandle = FileHandle(forReadingAtPath: url.relativePath),
            let metadata = ContainerMetadata(fileHandle: fileHandle)
        else {
            return nil
        }
        let newPath = storageDirectoryPath + "/" + metadata.sha256 + ".webc"
        do {
            try fileManager.copyItem(atPath: url.relativePath, toPath: newPath)
        } catch {
            return nil
        }

        updateRefs()
        return refs.first { $0.metadata.sha256 == metadata.sha256 }
     }
}
