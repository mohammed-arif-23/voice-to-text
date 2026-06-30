namespace Desktop.Core;

public record TranscriptSegment(
    string Text,
    double Timestamp,
    SegmentKind Kind
);
