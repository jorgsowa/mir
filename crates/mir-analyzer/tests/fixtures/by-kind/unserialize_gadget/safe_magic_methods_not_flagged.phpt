===description===
Constant arguments, escaped properties, output, ordinary methods, and other magic
methods aren't gadgets.
===config===
<mir>
  <issueHandlers>
    <UnserializeGadget errorLevel="warning"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Lock {
    private string $name = '';
    public function __destruct() {
        unlink(sys_get_temp_dir() . '/app.lock');
        shell_exec('notify ' . escapeshellarg($this->name));
        echo $this->name;
    }
    public function release(): void {
        unlink($this->name);
    }
    public function __construct(string $name) {
        $this->name = $name;
        touch($this->name);
    }
}
