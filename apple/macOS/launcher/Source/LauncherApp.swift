import SwiftUI
import WebrogueCommon

@main
struct LauncherApp: App {
    static var containerStorage = ContainerStorage()
    @ObservedObject var containerStorage = Self.containerStorage

    var documentGroup: some Scene {
        let result = DocumentGroup(viewing: ContainerDocument.self) { file in
            if
                let url = file.fileURL,
                let metadata = ContainerMetadata(url: url)
            {
                ContainerReferenceView(
                    for: ContainerReference(
                        path: url.relativePath,
                        metadata: metadata
                    )
                )
            } else {
                Text("This file cant be opened")
            }
        }

        if #available(macOS 13, *) {
            return result.commandsRemoved()
        } else {
            return result
                .commands {
                    CommandGroup(replacing: .saveItem) {}
                }
        }
    }

    var body: some Scene {
//        WindowGroup {
//            ContainerListView()
//        }
        documentGroup
            .commands {
                CommandGroup(replacing: .undoRedo) {}
            }
    }
}
