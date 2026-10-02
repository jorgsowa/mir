===description===
hash() returns non-empty-string; max() with a non-negative literal and an int
returns a non-negative range.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
final class Ids {
    /** @var non-empty-string */
    private string $id = 'x';

    public function set(string $data, mixed $raw): void {
        $id = hash('sha256', $data);
        /** @mir-check $id is non-empty-string */
        $this->id = $id;

        $n = max(0, (int) $raw);
        /** @mir-check $n is non-negative-int */
        $_ = $n;
    }
}
===expect===
