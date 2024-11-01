import SwiftUI
import WebrogueCommon

@main
struct LauncherApp: App {
    static var wrappStorage = WrappStorage()
    @ObservedObject var wrappStorage = Self.wrappStorage

    var documentGroup: some Scene {
        let result = DocumentGroup(viewing: WrappDocument.self) { file in
            if
                let url = file.fileURL,
                let metadata = WrappMetadata(url: url)
            {
                WrappRefView(
                    for: WrappRef(
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
//            WrappListView()
//        }
        documentGroup
            .commands {
                CommandGroup(replacing: .undoRedo) {}
            }
    }
}
