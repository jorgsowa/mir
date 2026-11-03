===description===
A `self::` call to a method the winning declaration of a duplicated class lacks still counts
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file:a.php===
<?php
final class Combiner {
    public static function combine(array $a): int {
        $n = 0;
        foreach ($a as $x) {
//                     ^^ MixedAssignment: Variable $x is assigned a mixed type
            foreach ($a as $y) {
//                         ^^ MixedAssignment: Variable $y is assigned a mixed type
                $n += self::intersect($x, $y);
//              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $n is assigned a mixed type
//                    ^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Combiner::intersect() does not exist
            }
        }
        return $n;
//      ^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
    }

    private static function intersect(mixed $x, mixed $y): int { return $x === $y ? 1 : 0; }
//                                                                      ^^ UndefinedVariable: Variable $x is not defined
//                                                                             ^^ UndefinedVariable: Variable $y is not defined
}
===file:b.php===
<?php
final class Combiner {
    public static function combine(array $a): int { return 0; }
//                                 ^^^^^^^^ UnusedParam: Parameter $a is never used
}
Combiner::combine([]);
