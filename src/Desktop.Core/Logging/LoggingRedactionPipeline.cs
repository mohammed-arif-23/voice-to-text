using System;
using System.Collections.Generic;

namespace Desktop.Logging;

public class LoggingRedactionPipeline
{
    public static string RedactValue(string propertyName, string value)
    {
        if (propertyName.Equals("Transcript", StringComparison.OrdinalIgnoreCase) ||
            propertyName.Equals("ClipboardContent", StringComparison.OrdinalIgnoreCase) ||
            propertyName.Equals("ApiToken", StringComparison.OrdinalIgnoreCase))
        {
            return "***";
        }

        if (propertyName.Equals("WindowTitle", StringComparison.OrdinalIgnoreCase))
        {
            using var sha256 = System.Security.Cryptography.SHA256.Create();
            byte[] bytes = sha256.ComputeHash(System.Text.Encoding.UTF8.GetBytes(value));
            return Convert.ToHexString(bytes).ToLowerInvariant();
        }

        if (propertyName.Equals("FilePath", StringComparison.OrdinalIgnoreCase) || propertyName.Equals("StackTrace", StringComparison.OrdinalIgnoreCase))
        {
            return System.Text.RegularExpressions.Regex.Replace(value, @"(?i)(C:\\Users\\)[^\\]+(\\)", "$1***$2");
        }

        return value;
    }

    public static object RedactObject(object obj)
    {
        if (obj is Dictionary<string, string> dict)
        {
            var newDict = new Dictionary<string, string>();
            foreach (var (k, v) in dict)
            {
                if (k.Contains("Redacted") || k.Equals("Token", StringComparison.OrdinalIgnoreCase))
                {
                    newDict[k] = "***";
                }
                else
                {
                    newDict[k] = v;
                }
            }
            return newDict;
        }
        return obj;
    }
}
