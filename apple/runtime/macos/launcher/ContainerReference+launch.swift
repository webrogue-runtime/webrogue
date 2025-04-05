import Foundation

extension ContainerReference {
    func launch(
        stdoutHandler: @escaping (Data) -> Void,
        terminatorSetter: (@escaping () -> Void) -> Void
    ) async {
        var pathComponents = Bundle.main.executablePath!.components(separatedBy: "/")
        pathComponents.removeLast()
        pathComponents.append("webrogue_runtime")

        let task = Process()
        let pipe = Pipe()

        task.standardInput = nil
        task.standardOutput = pipe
        task.standardError = pipe

        task.arguments = [
            path,
            dataPath,
        ]
        task.launchPath = "/" + pathComponents.joined(separator: "/")
        pipe.fileHandleForReading.readabilityHandler = { fileHandle in
            stdoutHandler(fileHandle.availableData)
        }
        terminatorSetter({
            task.terminate()
        })

        await withCheckedContinuation { continuation in
            task.terminationHandler = { _ in
                continuation.resume()
            }
            task.launch()
        }
    }
}
