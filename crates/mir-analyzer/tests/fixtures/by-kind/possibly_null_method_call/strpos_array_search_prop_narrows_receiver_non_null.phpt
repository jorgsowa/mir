===description===
`strpos($obj->prop, 'needle') !== false` / `array_search($obj->prop,
$haystack, true) !== false` must also prove `$obj` itself is non-null —
same reasoning as the already-fixed `in_array()` sibling: a found result
proves the property read wasn't null-derived. The not-found direction
(last two functions) proves nothing about the receiver.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    public ?string $tag = null;
    public function ping(): void {}
}

function viaStrpos(?Box $b): void {
    if (strpos($b->tag, 'needle') !== false) {
//             ^^^^^^^ PossiblyNullArgument: Argument $haystack of strpos() might be null
//             ^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $tag on possibly null value
        $b->ping();
    }
}

function viaArraySearch(?Box $b): void {
    if (array_search($b->tag, ['a', 'b', 'c'], true) !== false) {
//                   ^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $tag on possibly null value
        $b->ping();
    }
}

// Negative: not-found proves nothing about $b itself.
function viaStrposNotFound(?Box $b): void {
    if (strpos($b->tag, 'needle') === false) {
//             ^^^^^^^ PossiblyNullArgument: Argument $haystack of strpos() might be null
//             ^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $tag on possibly null value
        $b->ping();
//      ^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method ping() on possibly null value
    }
}

function viaArraySearchNotFound(?Box $b): void {
    if (array_search($b->tag, ['a', 'b', 'c'], true) === false) {
//                   ^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $tag on possibly null value
        $b->ping();
//      ^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method ping() on possibly null value
    }
}
