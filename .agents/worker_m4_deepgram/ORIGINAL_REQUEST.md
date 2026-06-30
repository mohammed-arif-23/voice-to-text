## 2026-06-30T11:25:03Z
You are the Worker for implementing the Deepgram Nova-3 Adapter.
Your working directory is `/Users/mohammedarif/voice-to-text/.agents/worker_m4_deepgram`.

Objectives:
1. Implement the `DeepgramTranscriptionProvider` class in `src/Desktop.Transcription/DeepgramTranscriptionProvider.cs` (namespace `Desktop.Transcription`).
2. It must implement the `Desktop.Core.IStreamingTranscriptionProvider` interface.
3. In `ConnectAsync`, establish a ClientWebSocket connection to `wss://api.deepgram.com/v1/listen?model=nova-3&encoding=linear16&sample_rate=16000&interim_results=true&endpointing=200&keyterm=patient`. The token must be passed in the `Authorization` request header as `Token <token>`. Throw `ProviderAuthException` if the token is null/empty/invalid, or if the connection fails due to authentication or network reasons.
4. Implement a background keep-alive loop that sends `{ "type": "KeepAlive" }` as a text message to the WebSocket every 10 seconds while connected.
5. In `SendAudioAsync`, send the audio data from the `AudioFrame` over the WebSocket as a binary message. If a socket/network exception occurs, raise the `ErrorOccurred` event and propagate the exception.
6. In `DisconnectAsync`, send `{ "type": "CloseStream" }` as a text message to the WebSocket, and then gracefully close the connection.
7. Start a background receive loop upon connecting. For each message received from the WebSocket:
   - Parse the JSON payload.
   - Extract `is_final` and the transcript from `channel.alternatives[0].transcript`.
   - Raise the `SegmentReceived` event with a `TranscriptSegment` (text, timestamp, SegmentKind.Final/Interim).
8. Write a unit/contract test class `tests/Contract/Transcription/DeepgramContractTests.cs` to test these behaviors. Use mock WebSockets or HttpMessageHandler to verify WSS connection headers, keep-alive sending, close stream sending, and parsing of mock JSON payloads. You can create a mock fixture at `tests/Contract/Fixtures/deepgram_response.json` for testing the parser.
9. Ensure that `src/Desktop.Transcription` compiles successfully in Release configuration with zero warnings and the contract tests pass.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Finally, reply back to parent with your handoff path and build/test results.
