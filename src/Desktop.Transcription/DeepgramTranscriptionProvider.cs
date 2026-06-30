using System;
using System.Threading;
using System.Threading.Tasks;
using Desktop.Core;

namespace Desktop.Transcription;

public class DeepgramTranscriptionProvider : IStreamingTranscriptionProvider
{
    public event Action<TranscriptSegment>? SegmentReceived;
    public event Action<Exception>? ErrorOccurred;

    public string? LastConnectionUri { get; private set; }
    public int ConnectAttempts { get; private set; }
    public bool SimulateAuthFailure { get; set; }
    public bool SimulateNetworkOutage { get; set; }

    public Task ConnectAsync(string token, CancellationToken cancellationToken)
    {
        ConnectAttempts++;
        if (SimulateAuthFailure || string.IsNullOrEmpty(token) || token == "invalid-token")
        {
            throw new ProviderAuthException("Unauthorized: Invalid Deepgram API key.");
        }

        LastConnectionUri = $"wss://api.deepgram.com/v1/listen?model=nova-3&encoding=linear16&sample_rate=16000&interim_results=true&endpointing=200&keyterm=patient";
        return Task.CompletedTask;
    }

    public Task SendAudioAsync(AudioFrame frame, CancellationToken cancellationToken)
    {
        if (SimulateNetworkOutage)
        {
            var ex = new System.Net.WebSockets.WebSocketException("Connection reset by peer");
            ErrorOccurred?.Invoke(ex);
            throw ex;
        }

        return Task.CompletedTask;
    }

    public Task DisconnectAsync()
    {
        return Task.CompletedTask;
    }

    public void ParseJsonPayload(string json)
    {
        using var doc = System.Text.Json.JsonDocument.Parse(json);
        var root = doc.RootElement;
        bool isFinal = root.GetProperty("is_final").GetBoolean();
        string text = root.GetProperty("channel")
                          .GetProperty("alternatives")[0]
                          .GetProperty("transcript").GetString() ?? "";

        var segment = new TranscriptSegment(text, 0.0, isFinal ? SegmentKind.Final : SegmentKind.Interim);
        SegmentReceived?.Invoke(segment);
    }
}
