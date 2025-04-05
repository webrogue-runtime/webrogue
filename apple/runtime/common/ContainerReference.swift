public struct ContainerReference {
    public let path: String
    public let metadata: ContainerMetadata

    public init(path: String, metadata: ContainerMetadata) {
        self.path = path
        self.metadata = metadata
    }

    public var dataPath: String {
        let fileManager = FileManager.default
        let documentDirPath = fileManager.urls(
            for: .documentDirectory,
            in: .userDomainMask
        ).first!.relativePath
        let dataDirPath = documentDirPath + "/.webrogue"
        if !fileManager.fileExists(atPath: dataDirPath) {
            try! fileManager.createDirectory(atPath: dataDirPath, withIntermediateDirectories: true)
        }
        return dataDirPath + "/data/" + metadata.id
    }
}
