===description===
When @param int conflicts with bool hint, the PHP hint wins (bool). Passing a
string to a bool-hinted param still fires InvalidArgument.
===config===
suppress=UnusedParam,MismatchingDocblockParamType
===file===
<?php
class Converter {
    /**
     * @param int $withQuote
     */
    public static function convert(string $text, bool $withQuote = false): string {
        return $withQuote ? htmlspecialchars($text, ENT_QUOTES) : htmlspecialchars($text);
    }
}

Converter::convert('hello', 'yes');
===expect===
InvalidArgument@11:28-11:33: Argument $withQuote of convert() expects 'bool', got '"yes"'
