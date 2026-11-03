===description===
`[__CLASS__, 'method']` is an array callable naming the enclosing class
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class A {
    private static function compare(int $a, int $b): int {
        return $a <=> $b;
    }

    /** @param list<int> $items */
    public static function sort(array $items): array {
        usort($items, [__CLASS__, 'compare']);
        return $items;
    }
}

A::sort([2, 1]);
