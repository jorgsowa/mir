===description===
Magic methods that run on unserialized objects passing a property to a file,
shell, callback or SQL call.
===config===
<mir>
  <issueHandlers>
    <PossiblyInvalidArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class TempFile {
    private string $path = '';
    public function __destruct() {
        unlink($this->path);
//      ^^^^^^^^^^^^^^^^^^^ UnserializeGadget: TempFile::__destruct() passes a property to a dangerous call; an unserialized object controls that property
    }
}

final class Job {
    private string $command = '';
    /** @var callable */
    private $onWake;
    public function __wakeup(): void {
        call_user_func($this->onWake);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnserializeGadget: Job::__wakeup() passes a property to a dangerous call; an unserialized object controls that property
    }
    public function __toString(): string {
        return (string) shell_exec('run ' . $this->command);
//                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnserializeGadget: Job::__toString() passes a property to a dangerous call; an unserialized object controls that property
    }
}

final class Report {
    private string $table = '';
    /** @param array<string, string> $data */
    public function __unserialize(array $data): void {
        $this->table = $data['table'];
        mysqli_query(mysqli_connect(), "DELETE FROM {$this->table}");
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnserializeGadget: Report::__unserialize() passes a property to a dangerous call; an unserialized object controls that property
    }
}
