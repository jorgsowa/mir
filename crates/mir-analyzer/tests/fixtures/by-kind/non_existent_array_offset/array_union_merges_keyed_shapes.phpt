===description===
`array + array` over keyed shapes keeps left keys and appends right-only keys
(including class-constant shapes), instead of keeping only the left shape.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    const DEFAULTS = ['un_a' => 1, 'un_b' => 2];

    public function viaSelf(): int {
        $o = ['un_c' => 3] + self::DEFAULTS;
        /** @mir-check $o is array{'un_c': 3, 'un_a': 1, 'un_b': 2} */
        return $o['un_a'] + $o['un_c'];
    }

    public function viaStatic(): int {
        $o = ['un_c' => 3] + static::DEFAULTS;
        return $o['un_b'];
    }

    public function compound(): int {
        $o = ['un_c' => 3];
        $o += self::DEFAULTS;
        return $o['un_a'];
    }

    public function leftWins(): int {
        $o = ['un_a' => 'x'] + self::DEFAULTS;
        /** @mir-check $o is array{'un_a': 'x', 'un_b': 2} */
        return $o['un_b'];
    }

    public function plainArrayRight(array $x): int {
        /** @var array<string, int> $x */
        $o = ['un_c' => 3] + $x;
        return $o['un_c'] + $o['anything'];
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
    }
}

function fromFunction(): int {
    $o = ['un_c' => 3] + A::DEFAULTS;
    return $o['un_b'];
}

function stillReportsMissingKey(): int {
    $o = ['un_c' => 3] + A::DEFAULTS;
    return $o['un_zzz'];
//  ^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//            ^^^^^^^^ NonExistentArrayOffset: Array offset 'un_zzz' does not exist
}
