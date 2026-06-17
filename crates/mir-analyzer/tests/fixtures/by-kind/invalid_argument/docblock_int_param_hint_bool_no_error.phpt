===description===
When @param int conflicts with a bool PHP hint, the PHP hint is the runtime
truth. Passing a bool (e.g. isset()) must NOT fire InvalidArgument.
===config===
suppress=UnusedParam,MismatchingDocblockParamType
===file===
<?php
class Converter {
    /**
     * @param int $withQuote   <-- wrong docblock, PHP hint is bool
     */
    public static function convert(string $text, bool $withQuote = false): string {
        return $withQuote ? htmlspecialchars($text, ENT_QUOTES) : htmlspecialchars($text);
    }
}

class Mailer {
    public bool $hasMarkdown = true;

    public function render(): string {
        return Converter::convert('hello', withQuote: isset($this->hasMarkdown));
    }
}
===expect===
