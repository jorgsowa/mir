===description===
str_contains($haystack, $this->needle) narrows the haystack to
non-empty-string when the needle is a property already narrowed to a
single non-empty string literal, same as a plain variable needle already
does — needle_non_empty resolution only tried extract_var_name.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor,MissingThrowsDocblock
===file===
<?php
class Holder {
    /** @var 'x' */
    public string $needle = 'x';
    public string $emptyNeedle = '';
}

function test_str_contains_prop_needle(string $haystack, Holder $h): void {
    if (str_contains($haystack, $h->needle)) {
        /** @mir-check $haystack is non-empty-string */
        $_ = $haystack;
    }
}

function test_strpos_prop_needle(string $haystack, Holder $h): void {
    if (strpos($haystack, $h->needle) !== false) {
        /** @mir-check $haystack is non-empty-string */
        $_ = $haystack;
    }
}

function test_empty_prop_needle_not_narrowed(string $haystack, Holder $h): void {
    if (str_contains($haystack, $h->emptyNeedle)) {
        /** @mir-check $haystack is string */
        $_ = $haystack;
    }
}
===expect===
