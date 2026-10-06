===description===
A `@param Other::*` wildcard resolves to the constants' values, so it is a valid array key.
===file===
<?php
final class Kind {
    public const string A = 'a';
    public const string B = 'b';
}
final class Reader {
    /**
     * @param Kind::* $k
     * @param array<string, int> $map
     */
    public function read(string $k, array $map): int {
        $v = $map[$k];
        /** @mir-check $v is int */
        return $v;
    }
}
