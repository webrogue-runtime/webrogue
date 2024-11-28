package io.github.webrogue_runtime.launcher

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults.topAppBarColors
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import io.github.webrogue_runtime.launcher.theme.WebrogueTheme
import io.github.webrogue_runtime.common.container.ContainerFileManage
import io.github.webrogue_runtime.common.container.ContainerReference
import io.github.webrogue_runtime.common.WebrogueActivity
import java.util.function.Consumer

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun WrappCard(
    ref: ContainerReference,
    delete: () -> Unit,
    launch: () -> Unit
) {
    var dropdownExpanded by remember { mutableStateOf(false) }
    Surface(
        shape = MaterialTheme.shapes.medium,
        shadowElevation = 1.dp,
        modifier = Modifier
            .animateContentSize()
            .combinedClickable(
                onClick = launch,
                onLongClick = {
                    dropdownExpanded = true
                }
            )
    ) {
        DropdownMenu(
            expanded = dropdownExpanded,
            onDismissRequest = { dropdownExpanded = false }
        ) {
            DropdownMenuItem(
                text = {  Text("Run") },
                onClick = launch
            )
            DropdownMenuItem(
                text = { Text("Delete") },
                onClick = delete
            )
//            HorizontalDivider()
        }
        Column(
            horizontalAlignment = Alignment.Start,
            modifier = Modifier
                .fillMaxWidth()
                .padding(all = 16.dp)
        ) {
//            Row(
//                horizontalArrangement = Arrangement.SpaceBetween
//            ) {
//                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "sha: ${ref.sha}",
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
//                }
//            }
        }
    }
}

class WebrogueLauncherActivity : ComponentActivity() {
    private var containerFileManage: ContainerFileManage? = null
    private var onNewIntentListeners: Set<Consumer<Unit>> = setOf()

    override fun onNewIntent(intent: Intent?) {
        processIntent(intent)
        for (listener in onNewIntentListeners) {
            listener.accept(Unit)
        }
        super.onNewIntent(intent)
    }
    private fun processIntent(intent: Intent?) {
        if(intent != null && intent.action != Intent.ACTION_MAIN && intent.type != null) {
            val uri = intent.data ?: return
            val stream = contentResolver.openInputStream(uri) ?: return
            containerFileManage?.storeWrappFromStream(stream)
        }
    }
    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        containerFileManage = ContainerFileManage(filesDir)
        super.onCreate(savedInstanceState)
        processIntent(intent)
        enableEdgeToEdge()
        setContent {
            var refs: List<ContainerReference> by remember { mutableStateOf(containerFileManage!!.listMods()) }
            WebrogueTheme {
                Scaffold(
                    topBar = {
                        TopAppBar(
                            colors = topAppBarColors(
                                containerColor = MaterialTheme.colorScheme.primaryContainer,
                                titleContentColor = MaterialTheme.colorScheme.primary,
                            ),
                            title = {
                                Text("webrogue")
                            }
                        )
                    }
                ) { innerPadding ->
                    LazyColumn(
                        modifier = Modifier.padding(innerPadding)
                    ) {
                        items(refs) { ref ->
                            WrappCard(
                                ref = ref,
                                delete = {
                                    ref.delete()
                                    refs = containerFileManage!!.listMods()
                                },
                                launch = {
                                    val myIntent = Intent(
                                        this@WebrogueLauncherActivity,
                                        WebrogueActivity::class.java
                                    )
                                    myIntent.putExtra("wrapp_path", ref.path);
                                    myIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                                    // myIntent.addFlags(Intent.FLAG_ACTIVITY_MULTIPLE_TASK);
                                    this@WebrogueLauncherActivity.startActivity(myIntent)
                                }
                            )
                        }
                    }
                }
            }
            DisposableEffect(1) {
                val listener = Consumer<Unit> {
                    refs = containerFileManage!!.listMods()
                }
                onNewIntentListeners += listener
                onDispose { onNewIntentListeners -= listener }
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    WebrogueTheme {
        Column(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                text = "Hello Android!"
            )
        }
    }
}
