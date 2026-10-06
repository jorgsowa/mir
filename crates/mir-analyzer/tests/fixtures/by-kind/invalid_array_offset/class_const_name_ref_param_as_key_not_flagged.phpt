===description===
A `@param Other::NAME` union and inherited constants are valid array keys.
===file===
<?php
class Base {
    public const string INHERITED = 'i';
}
final class Kind extends Base {
    public const string A = 'a';
    public const int N = 3;
}
final class Reader {
    /**
     * @param Kind::A|Kind::N $k
     * @param array<int|string, int> $map
     */
    public function single(string|int $k, array $map): int {
        $v = $map[$k];
        /** @mir-check $v is int */
        return $v;
    }

    /**
     * @param Kind::* $k
     * @param array<int|string, int> $map
     */
    public function all(string|int $k, array $map): int {
        $v = $map[$k];
        /** @mir-check $v is int */
        return $v;
    }
}
