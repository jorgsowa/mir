===description===
Passing an undefined variable to a by-reference method parameter (an out-param)
defines it, so it must not be reported as UndefinedVariable.
===file===
<?php
class T {
    private function arrange(array &$actual, array $all): int {
        $actual = $all;
        return count($all);
    }
    public function run(): int {
        $n = $this->arrange($captured, [1, 2, 3]);
        return count($captured) + $n;
    }
    public function genuinelyUndef(): void {
        echo $nope;
    }
}

===expect===
UndefinedVariable@12:13-12:18: Variable $nope is not defined
