===description===
Instance and static method first-class callables on the right of `|>` are
called with the left operand.
===file===
<?php
final class Text {
    public function length(string $s): int { return strlen($s); }
    /** @return list<int> */
    public static function wrap(int $i): array { return [$i]; }
    /** @return list<int> */
    public function viaSelf(int $i): array { return $i |> self::wrap(...); }
}

$a = "abc" |> (new Text())->length(...);
/** @trace $a */
$b = 3 |> Text::wrap(...);
/** @trace $b */
$c = 4 |> (new Text())->viaSelf(...);
/** @trace $c */
echo $a, count($b), count($c);
===expect===
Trace@12:0-12:26: Type of $a is int
Trace@14:0-14:37: Type of $b is list<int>
Trace@16:0-16:30: Type of $c is list<int>
