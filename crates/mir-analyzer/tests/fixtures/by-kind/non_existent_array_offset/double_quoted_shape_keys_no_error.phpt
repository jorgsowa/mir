===description===
Double-quoted shape keys in @param array{...} are treated the same as bare keys
===file===
<?php

class Mapper {
    /**
     * @param array{
     *  "id": int,
     *  "account": int,
     * } $row
     */
    public function map(array $row): void
    {
        echo $row['id'];
        echo $row['account'];
    }
}
===expect===
