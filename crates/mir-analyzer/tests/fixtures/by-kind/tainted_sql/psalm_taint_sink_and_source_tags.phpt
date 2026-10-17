===description===
Psalm's @psalm-taint-sink and @psalm-taint-source spellings behave like
@taint-sink and @taint-source.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Db {
    /** @psalm-taint-sink sql $sql */
    public function run(string $sql): void {
    }
}

class Input {
    /** @psalm-taint-source input */
    public function get(string $name): string {
        return '';
    }
}

function handle(Db $db, Input $in): void {
    $db->run((string) $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    $db->run($in->get('q'));
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    $db->run('SELECT 1');
}
