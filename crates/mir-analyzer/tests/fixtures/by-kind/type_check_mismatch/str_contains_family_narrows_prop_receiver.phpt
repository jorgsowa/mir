===description===
str_contains/str_starts_with/str_ends_with with a non-empty literal needle
narrow a property-access haystack to non-empty-string too, not just a plain
variable.
===config===
suppress=UnusedVariable,UnusedParam,MissingThrowsDocblock
===file===
<?php
class Holder {
    public string $text = '';
}

function test_str_contains_prop(Holder $h): void {
    if (str_contains($h->text, 'x')) {
        /** @mir-check $h->text is non-empty-string */
        $_ = $h->text;
    }
}

function test_str_starts_with_prop(Holder $h): void {
    if (str_starts_with($h->text, 'prefix')) {
        /** @mir-check $h->text is non-empty-string */
        $_ = $h->text;
    }
}

function test_str_ends_with_prop(Holder $h): void {
    if (str_ends_with($h->text, 'suffix')) {
        /** @mir-check $h->text is non-empty-string */
        $_ = $h->text;
    }
}

function test_empty_needle_prop_not_narrowed(Holder $h): void {
    if (str_contains($h->text, '')) {
        /** @mir-check $h->text is string */
        $_ = $h->text;
    }
}
===expect===
