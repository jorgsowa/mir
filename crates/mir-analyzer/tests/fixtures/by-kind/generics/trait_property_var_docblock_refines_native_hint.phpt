===description===
FN: a trait property's @var docblock was ignored entirely — only the
native type hint was ever used, unlike the equivalent class property.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
trait HasCount {
    /** @var int */
    public mixed $count;
}
class Foo {
    use HasCount;

    public function test(): void {
        $c = $this->count;
        /** @mir-check $c is int */
        echo '';
    }
}
===expect===
