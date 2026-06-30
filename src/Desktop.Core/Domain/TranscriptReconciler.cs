using System;
using System.Collections.Generic;
using System.Linq;

namespace Desktop.Core;

public class TranscriptReconciler
{
    private readonly List<TranscriptSegment> _stableSegments = new();
    private readonly VoiceCommandParser _commandParser = new();

    public IReadOnlyList<TranscriptSegment> StableSegments => _stableSegments;

    public string AddSegment(TranscriptSegment segment, string locale = "en-US")
    {
        if (segment.Kind == SegmentKind.Final)
        {
            var controlSignals = new List<string>();
            string parsedText = _commandParser.Parse(segment.Text, out controlSignals, locale);

            var finalSegment = segment with { Text = parsedText };
            _stableSegments.Add(finalSegment);
        }

        return GetReconciledText();
    }

    public string ReconcileInterims(List<TranscriptSegment> interims)
    {
        var latestInterim = interims
            .Where(x => x.Kind == SegmentKind.Interim)
            .OrderByDescending(x => x.Timestamp)
            .FirstOrDefault();

        string stableText = GetReconciledText();
        if (latestInterim != null)
        {
            var dummySignals = new List<string>();
            string interimText = _commandParser.Parse(latestInterim.Text, out dummySignals);
            if (!string.IsNullOrEmpty(interimText))
            {
                if (!string.IsNullOrEmpty(stableText) && interimText.StartsWith(stableText, StringComparison.OrdinalIgnoreCase))
                {
                    interimText = interimText[stableText.Length..].TrimStart();
                    return string.IsNullOrEmpty(interimText) ? stableText : $"{stableText} {interimText}";
                }
                return string.IsNullOrEmpty(stableText) ? interimText : $"{stableText} {interimText}";
            }
        }
        return stableText;
    }

    public string GetReconciledText()
    {
        return string.Join(" ", _stableSegments.Select(s => s.Text).Where(t => !string.IsNullOrWhiteSpace(t)));
    }

    public void Clear()
    {
        _stableSegments.Clear();
    }
}
