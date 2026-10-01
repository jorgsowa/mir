===description===
Variadic `int ...$ids` against a parent `@param int[] $ids` compares element types; a non-variadic or mismatched element still flags.
===config===
suppress=UnusedParam
===file===
<?php
interface Reader {
    /** @param int[] $ids */
    public function byIds(int ...$ids): array;
    /** @param list<int> $ids */
    public function byList(int ...$ids): array;
    /** @param int ...$ids */
    public function bare(int ...$ids): array;
    /** @param int[] $ids */
    public function nested(array ...$ids): array;
    /** @param int[] $ids */
    public function strict(int ...$ids): array;
}
final class ReadService implements Reader {
    /** @inheritDoc */
    public function byIds(int ...$ids): array { return []; }
    /** @inheritDoc */
    public function byList(int ...$ids): array { return []; }
    public function bare(int ...$ids): array { return []; }
    public function nested(array ...$ids): array { return []; }
    public function strict(string ...$ids): array { return []; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method ReadService::strict() signature mismatch: parameter $ids type 'string' is incompatible with parent type 'int'
}
===expect===
