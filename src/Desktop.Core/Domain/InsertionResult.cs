namespace Desktop.Core;

public record InsertionResult(
    bool Success,
    string InsertedText,
    AdapterKind RouteChosen,
    string? ErrorMessage = null
);
