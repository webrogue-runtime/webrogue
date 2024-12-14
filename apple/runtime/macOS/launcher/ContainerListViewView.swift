import SwiftUI

struct ContainerListView: View {
    @State var isFileImporterPresented = false

    @ObservedObject var containerStorage = LauncherApp.containerStorage

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
            allowedContentTypes: [.webc],
            allowsMultipleSelection: false
        ) { result in
            switch result {
            case .success(let files):
                for file in files {
                    let gotAccess = file.startAccessingSecurityScopedResource()
                    if !gotAccess { continue }
                    containerStorage.store(file)
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
    ContainerListView()
}
