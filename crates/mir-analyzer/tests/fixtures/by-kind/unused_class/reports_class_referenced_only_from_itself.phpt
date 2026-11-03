===description===
References from inside a class's own body don't keep it alive
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Factory {
//    ^ +4:1 UnusedClass: Class Factory is never referenced
    public static function make(): self {
        return new Factory();
    }
}
