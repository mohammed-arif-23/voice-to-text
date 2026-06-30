using System;

namespace Desktop.Core;

public record SessionId(Guid Value)
{
    public static SessionId New() => new(Guid.NewGuid());
    public override string ToString() => Value.ToString();
}
