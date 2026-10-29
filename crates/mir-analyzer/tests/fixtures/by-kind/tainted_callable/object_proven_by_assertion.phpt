===description===
A parameter an assertion proves is an object (an inferred `instanceof` predicate or a declared `@psalm-assert-if-true`) carries request data only if the caller passed an object, as after an inline `instanceof`; other assertions keep the flow.
===file===
<?php
class Query {
    protected function isQueryable(mixed $value): bool { return $value instanceof self || $value instanceof Closure; }

    public function where(mixed $value): void {
        if ($this->isQueryable($value)) {
            $value($this);
        }
    }
}

/** @psalm-assert-if-true Closure $v */
function isCallback(mixed $v): bool { return is_object($v); }

/** @psalm-assert-if-true string $v */
function isName(mixed $v): bool { return is_string($v); }

function declared(mixed $value): void {
    if (isCallback($value)) {
        $value(1);
    }
}

function notObject(mixed $value): void {
    if (isName($value)) {
        $value(1);
    }
}

function negated(mixed $value): void {
    if (!isCallback($value)) {
        $value(1);
//      ^^^^^^^^^ MixedFunctionCall: Cannot call mixed type as a function
    }
}

function callers(Query $q): void {
    $token = $_GET['x'];
//  ^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $token is assigned a mixed type
//           ^^^^^^^^^^ MixedArrayAccess: Array access on mixed type
    $q->where($token);
    declared($token);
    notObject($token);
//  ^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    negated($token);
//  ^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
}
