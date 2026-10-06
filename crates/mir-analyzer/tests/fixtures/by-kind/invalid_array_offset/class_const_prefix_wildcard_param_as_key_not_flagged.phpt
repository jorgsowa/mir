===description===
A `@param Other::PREFIX_*` wildcard covers only the matching constants.
===file===
<?php
final class Kind {
    public const string FOO_A = 'a';
    public const string FOO_B = 'b';
    public const string BAR = 'c';
}
final class Reader {
    /**
     * @param Kind::FOO_* $k
     * @param array<string, int> $map
     */
    public function read(string $k, array $map): int {
        $v = $map[$k];
        /** @mir-check $v is int */
        return $v;
    }
}
