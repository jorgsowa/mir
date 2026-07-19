===description===
strpos()'s haystack and array_search()'s needle narrowing (the property
counterparts of strpos_family_not_false_narrows_haystack.phpt and
array_search_not_false_narrows_needle.phpt) — both extractors only ever
recognized a plain-variable argument, not a property receiver.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
final class Holder {
    public string $s = '';
    public string $mode = '';

    public function strposNarrowsHaystack(): void {
        if (strpos($this->s, 'x') !== false) {
            /** @mir-check $this->s is non-empty-string */
            $_ = 1;
        }
    }

    public function arraySearchNarrowsNeedle(): void {
        if (array_search($this->mode, ['read', 'write', 'append']) !== false) {
            /** @mir-check $this->mode is "read"|"write"|"append" */
            $_ = 1;
        }
    }
}
===expect===
