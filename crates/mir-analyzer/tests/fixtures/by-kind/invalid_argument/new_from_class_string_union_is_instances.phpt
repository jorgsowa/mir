===description===
`new $class` where every member of `$class` is a known `class-string<X>` creates an
instance of one of the X; any other class-name type still yields `object`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Det {}
final class H implements Det {}
final class P implements Det {}

/** @template T of object */
final class Factory {
    /** @param class-string<T> $class */
    public function make(string $class): object {
        $o = new $class();
        /** @mir-check $o is object */
        return $o;
    }
}

/** @param iterable<Det> $d */
function takeDets(iterable $d): void {}
function takeP(P $p): void {}

/** @param list<string> $wanted */
function inLoopBranch(array $wanted): void {
    $out = [];
    foreach (['h' => H::class, 'p' => P::class] as $name => $class) {
        if (in_array($name, $wanted)) {
            $out[] = new $class();
        }
    }
    takeDets($out);
}

function ternary(bool $f): void {
    $class = $f ? H::class : P::class;
    $o = new $class();
    /** @mir-check $o is H|P */
    takeDets([$o]);
}

/** @param class-string<Det> $class */
function bound(string $class): void {
    $o = new $class();
    /** @mir-check $o is Det */
    takeDets([$o]);
}

function stillObject(string $name, bool $f): void {
    $o = new $name();
    /** @mir-check $o is object */
    $mixed = $f ? H::class : $name;
    $m = new $mixed();
    /** @mir-check $m is object */
    echo get_class($o), get_class($m);
}

function wrongMember(): void {
    $class = H::class;
    takeP(new $class());
//        ^^^^^^^^^^^^ InvalidArgument: Argument $p of takeP() expects 'P', got 'H'
}