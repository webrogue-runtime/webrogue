import SwiftUI
import WebrogueCommon

struct WrappListView: View {
    @State var isFileImporterPresented = false

    @ObservedObject var wrappStorage = LauncherApp.wrappStorage

    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Hello, world!")
            Button(action: {
                isFileImporterPresented = true
            }, label: {
                Text("Hello, world!")
            })
        }
        .fileImporter(
            isPresented: $isFileImporterPresented,
            allowedContentTypes: [.wrappType],
            allowsMultipleSelection: false
        ) { result in
            switch result {
            case .success(let files):
                for file in files {
                    let gotAccess = file.startAccessingSecurityScopedResource()
                    if !gotAccess { continue }
                    wrappStorage.store(file)
                    file.stopAccessingSecurityScopedResource()
                }
            case .failure(_):
                break
            }
        }
        .padding()
    }
}

#Preview {
    WrappListView()
}
