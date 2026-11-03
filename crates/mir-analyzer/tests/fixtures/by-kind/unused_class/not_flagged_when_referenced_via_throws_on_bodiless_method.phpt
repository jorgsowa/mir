===description===
`@throws` on interface and abstract methods references the exception class
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class EncodeFailed extends \Exception {}
final class DecodeFailed extends \Exception {}

interface Encoder {
    /** @throws EncodeFailed */
    public function encode(string $s): string;
}

abstract class Decoder {
    /** @throws DecodeFailed */
    abstract public function decode(string $s): string;
}

final class Codec extends Decoder implements Encoder {
    public function encode(string $s): string { return $s; }
    public function decode(string $s): string { return $s; }
}

new Codec();
