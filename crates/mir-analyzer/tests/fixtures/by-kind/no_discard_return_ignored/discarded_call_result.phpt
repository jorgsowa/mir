===description===
Discarding the result of a `#[\NoDiscard]` function or method warns on PHP 8.5; using it or casting to `(void)` doesn't.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
#[\NoDiscard("the id is unique")]
function mint(): int
{
    return 1;
}

function ordinary(): int
{
    return 1;
}

final class Svc
{
    #[\NoDiscard]
    public function make(): int
    {
        return 1;
    }

    #[\NoDiscard]
    public static function build(): int
    {
        return 1;
    }
}

function f(Svc $s, ?Svc $n, DateTimeImmutable $d): int
{
    mint();
//  ^^^^^^ NoDiscardReturnIgnored: The return value of function mint() should either be used or intentionally ignored by casting it as (void), the id is unique
    $x = mint();
    (void) mint();
    ordinary();
    $s->make();
//  ^^^^^^^^^^ NoDiscardReturnIgnored: The return value of method Svc::make() should either be used or intentionally ignored by casting it as (void)
    $n?->make();
//  ^^^^^^^^^^^ NoDiscardReturnIgnored: The return value of method Svc::make() should either be used or intentionally ignored by casting it as (void)
    Svc::build();
//  ^^^^^^^^^^^^ NoDiscardReturnIgnored: The return value of method Svc::build() should either be used or intentionally ignored by casting it as (void)
    $d->add(new DateInterval('P1D'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NoDiscardReturnIgnored: The return value of method DateTimeImmutable::add() should either be used or intentionally ignored by casting it as (void), as DateTimeImmutable::add() does not modify the object itself

    return $x + mint();
}
