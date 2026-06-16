===description===
Variables built via .= (concat-assign) must not be reported as unused — the LHS is read
on each compound-assignment, so the initial write is consumed.
===file===
<?php
function buildForeignKey(string $table, string $ref): string {
    $sql = "alter table {$table} add constraint fk ";
    $sql .= "foreign key (id) references {$ref} (id)";
    if (false) {
        $sql .= " on delete cascade";
    }
    return $sql;
}
===expect===
