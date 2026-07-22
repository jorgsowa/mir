===description===
`ScalarArgTarget::extract` (backing count()/array_key_first()/array_key_last()/
isset()/empty()/array_key_exists() shape narrowing on a property) only
recognized a plain `->` property receiver, never `?->` — a nullsafe access
disabled all of that narrowing family entirely.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,PossiblyNullArgument,PossiblyNullPropertyFetch
===file===
<?php
final class Holder {
    /** @var array|null */
    public $items;
}

function narrowsCountNullsafe(Holder $h): void {
    if (count($h?->items) > 0) {
        /** @mir-check $h->items is non-empty-array */
        $_ = $h->items;
    }
}

function narrowsIssetShapeNullsafe(Holder $h): void {
    if (isset($h?->items['key'])) {
        /** @mir-check $h->items is array */
        $_ = $h->items;
    }
}

function countPlainArrowStillWorks(Holder $h): void {
    if (count($h->items) > 0) {
        /** @mir-check $h->items is non-empty-array */
        $_ = $h->items;
    }
}
===expect===
