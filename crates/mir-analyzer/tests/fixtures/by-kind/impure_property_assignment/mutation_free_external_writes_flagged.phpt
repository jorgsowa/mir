===description===
`@mutation-free` ("nothing external") gave zero protection against a
static-property write, a whole-array superglobal write, an `unset()` of a
static property, or a by-ref-parameter write — every one of these
"external mutation" emitters gated strictly on `ctx.is_in_pure_fn`, never
consulting `ctx.is_in_immutable_method` (the flag `@mutation-free`
actually sets), even though a plain `$this`-property write was already
correctly checked via a separate, dedicated arm. Deliberately does NOT
include a `global $x;` declaration: that check fires regardless of
whether the variable is later written or only read, so it stays scoped to
@pure's stricter no-external-dependency contract (see
`impure_global_immutable.phpt`), not extended here.
===config===
suppress=MissingConstructor
===file===
<?php
class Counter {
    public static int $count = 0;
    /** @var array<string,int> */
    public static array $store = [];

    /** @mutation-free */
    public function bumpStatic(): void {
        self::$count = 5;
    }

    /** @mutation-free */
    public function overwriteSession(): void {
        $_SESSION = [];
    }

    /** @mutation-free */
    public function clearStatic(): void {
        unset(self::$store['k']);
    }

    /** @mutation-free */
    public function mutateByRef(int &$n): void {
        $n = 5;
    }
}
===expect===
ImpureStaticPropertyAssignment@9:8-9:24: Assigning to static property Counter::$count in a @pure function
ImpureGlobalVariable@14:8-14:22: Using global variable $_SESSION in a @pure function
ImpureStaticPropertyAssignment@19:14-19:31: Assigning to static property Counter::$store in a @pure function
ImpureByRefAssignment@24:8-24:14: Assigning to by-reference parameter $n in a @pure function
