===description===
A private constant nothing reads is reported; public and protected ones are not
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Config {
    private const UNUSED = 1;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedConstant: Private constant Config::UNUSED is never used
    private const USED = 2;
    public const API = 3;
    protected const INHERITED = 4;

    public function get(): int {
        return self::USED;
    }
}

(new Config())->get();
