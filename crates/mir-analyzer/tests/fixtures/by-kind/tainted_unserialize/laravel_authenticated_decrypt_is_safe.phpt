===description===
Laravel's Encrypter checks a keyed MAC before it unserializes, so a client can't choose the payload.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Illuminate\Contracts\Encryption {
    interface Encrypter { public function decrypt(string $payload, bool $unserialize = true): mixed; }
}
namespace Illuminate\Encryption {
    class Encrypter implements \Illuminate\Contracts\Encryption\Encrypter {
        public function decrypt(string $payload, bool $unserialize = true): mixed {
            return $unserialize ? unserialize($payload) : $payload;
        }
        public function decryptString(string $payload): string {
            return (string) $this->decrypt($payload, false);
        }
    }
}
namespace App {
    function direct(\Illuminate\Encryption\Encrypter $e): void {
        $e->decrypt($_COOKIE['a']);
        echo $e->decryptString($_COOKIE['b']);
    }
    function contract(\Illuminate\Contracts\Encryption\Encrypter $e): void {
        $e->decrypt($_COOKIE['a']);
    }
}
