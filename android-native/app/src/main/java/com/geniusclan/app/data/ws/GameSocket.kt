package com.geniusclan.app.data.ws

import com.geniusclan.app.BuildConfig
import com.geniusclan.app.data.api.ApiClient
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import org.json.JSONObject
import android.os.Handler
import android.os.Looper
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

class GameSocket(
    private val onEvent: (JSONObject) -> Unit,
    private val onState: (String) -> Unit
) {
    private val client = OkHttpClient.Builder()
        .pingInterval(20, TimeUnit.SECONDS)
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .build()

    private val socket = AtomicReference<WebSocket?>(null)
    private val main = Handler(Looper.getMainLooper())

    private fun postEvent(obj: JSONObject) {
        main.post { onEvent(obj) }
    }

    private fun postState(s: String) {
        main.post { onState(s) }
    }

    fun connectQueue() {
        val token = ApiClient.accessToken
        if (token.isNullOrBlank()) {
            postState("error:not_logged_in")
            return
        }
        val url = "${BuildConfig.WS_BASE_URL}/api/v1/match/queue?token=${java.net.URLEncoder.encode(token, "UTF-8")}"
        open(url)
    }

    fun connectMatch(matchId: String) {
        val token = ApiClient.accessToken
        if (token.isNullOrBlank()) {
            postState("error:not_logged_in")
            return
        }
        val url =
            "${BuildConfig.WS_BASE_URL}/api/v1/ws/match/$matchId?token=${java.net.URLEncoder.encode(token, "UTF-8")}"
        open(url)
    }

    private fun open(url: String) {
        close()
        postState("connecting")
        val req = Request.Builder().url(url).build()
        val ws = client.newWebSocket(req, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                postState("open")
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                try {
                    postEvent(JSONObject(text))
                } catch (_: Exception) {
                }
            }

            override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
                webSocket.close(1000, null)
                postState("closing")
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                postState("closed")
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                postState("error:${t.message ?: "ws_failed"}")
            }
        })
        socket.set(ws)
    }

    fun send(json: JSONObject) {
        socket.get()?.send(json.toString())
    }

    fun joinQueue(matchType: String = "casual") {
        send(JSONObject().put("type", "join_queue").put("match_type", matchType))
    }

    fun resumeMatch(matchId: String) {
        send(JSONObject().put("type", "resume_match").put("match_id", matchId))
    }

    fun move(matchId: String, from: String, to: String, promotion: String? = null) {
        val o = JSONObject()
            .put("type", "move")
            .put("match_id", matchId)
            .put("from", from)
            .put("to", to)
        if (promotion != null) o.put("promotion", promotion) else o.put("promotion", JSONObject.NULL)
        send(o)
    }

    fun resign(matchId: String) {
        send(JSONObject().put("type", "resign").put("match_id", matchId))
    }

    fun offerDraw(matchId: String) {
        send(JSONObject().put("type", "offer_draw").put("match_id", matchId))
    }

    fun close() {
        socket.getAndSet(null)?.close(1000, "bye")
    }
}
