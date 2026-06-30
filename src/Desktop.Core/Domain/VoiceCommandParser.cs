using System;
using System.Collections.Generic;
using System.Linq;

namespace Desktop.Core;

public class VoiceCommandParser
{
    public string Parse(string input, out List<string> controlSignals, string locale = "en-US")
    {
        controlSignals = new List<string>();
        if (string.IsNullOrEmpty(input)) return "";

        string lowerInput = input.ToLowerInvariant();
        if (lowerInput.Contains("stop dictation"))
        {
            controlSignals.Add("stop");
            input = System.Text.RegularExpressions.Regex.Replace(input, @"\bstop dictation\b", "", System.Text.RegularExpressions.RegexOptions.IgnoreCase);
        }
        if (lowerInput.Contains("cancel dictation"))
        {
            controlSignals.Add("cancel");
            input = System.Text.RegularExpressions.Regex.Replace(input, @"\bcancel dictation\b", "", System.Text.RegularExpressions.RegexOptions.IgnoreCase);
        }

        var words = input.Split(new[] { ' ', '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries).ToList();
        var resultWords = new List<string>();

        int i = 0;
        while (i < words.Count)
        {
            if (i + 2 < words.Count && MatchPhrase(words[i], words[i + 1], words[i + 2], "delete", "last", "word"))
            {
                if (resultWords.Count > 0)
                {
                    resultWords.RemoveAt(resultWords.Count - 1);
                }
                i += 3;
            }
            else if (i + 2 < words.Count && MatchPhrase(words[i], words[i + 1], words[i + 2], "delete", "last", "sentence"))
            {
                DeleteLastSentence(resultWords);
                i += 3;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "scratch", "that"))
            {
                resultWords.Clear();
                i += 2;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "new", "line"))
            {
                resultWords.Add("\n");
                i += 2;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "new", "paragraph"))
            {
                resultWords.Add("\n\n");
                i += 2;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "full", "stop") && (locale == "en-US" || locale == "en-GB"))
            {
                resultWords.Add(". ");
                i += 2;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "question", "mark"))
            {
                resultWords.Add("? ");
                i += 2;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "exclamation", "point"))
            {
                resultWords.Add("! ");
                i += 2;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "exclamation", "mark"))
            {
                resultWords.Add("! ");
                i += 2;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "open", "quotes"))
            {
                resultWords.Add(" \"");
                i += 2;
            }
            else if (i + 1 < words.Count && MatchPhrase(words[i], words[i + 1], "close", "quotes"))
            {
                resultWords.Add("\" ");
                i += 2;
            }
            else if (MatchPhrase(words[i], "comma"))
            {
                resultWords.Add(", ");
                i++;
            }
            else if (MatchPhrase(words[i], "period") && (locale == "en-US" || locale == "en-GB"))
            {
                resultWords.Add(". ");
                i++;
            }
            else if (MatchPhrase(words[i], "colon"))
            {
                resultWords.Add(": ");
                i++;
            }
            else if (MatchPhrase(words[i], "semicolon"))
            {
                resultWords.Add("; ");
                i++;
            }
            else
            {
                resultWords.Add(words[i]);
                i++;
            }
        }

        var builder = new System.Text.StringBuilder();
        for (int k = 0; k < resultWords.Count; k++)
        {
            string w = resultWords[k];
            if (k > 0)
            {
                string prev = resultWords[k - 1];
                if (w == ", " || w == ". " || w == "? " || w == "! " || w == ": " || w == "; " || w == "\" " || w == "\n" || w == "\n\n")
                {
                    if ((w == "\n" || w == "\n\n") && builder.Length > 0 && builder[builder.Length - 1] == ' ')
                    {
                        builder.Length--;
                    }
                }
                else if (prev == "\n" || prev == "\n\n" || prev == " \"" || prev.EndsWith(" ") || prev.EndsWith("\n"))
                {
                    // no space
                }
                else
                {
                    builder.Append(' ');
                }
            }
            builder.Append(w);
        }

        return builder.ToString();
    }

    private static bool MatchPhrase(string w1, string t1) =>
        string.Equals(w1, t1, StringComparison.OrdinalIgnoreCase);

    private static bool MatchPhrase(string w1, string w2, string t1, string t2) =>
        string.Equals(w1, t1, StringComparison.OrdinalIgnoreCase) &&
        string.Equals(w2, t2, StringComparison.OrdinalIgnoreCase);

    private static bool MatchPhrase(string w1, string w2, string w3, string t1, string t2, string t3) =>
        string.Equals(w1, t1, StringComparison.OrdinalIgnoreCase) &&
        string.Equals(w2, t2, StringComparison.OrdinalIgnoreCase) &&
        string.Equals(w3, t3, StringComparison.OrdinalIgnoreCase);

    private static void DeleteLastSentence(List<string> words)
    {
        int lastIdx = -1;
        for (int i = words.Count - 1; i >= 0; i--)
        {
            string w = words[i];
            if (w.Contains(". ") || w.Contains("? ") || w.Contains("! "))
            {
                lastIdx = i;
                break;
            }
        }
        if (lastIdx == -1)
        {
            words.Clear();
        }
        else
        {
            words.RemoveRange(lastIdx + 1, words.Count - (lastIdx + 1));
        }
    }
}
