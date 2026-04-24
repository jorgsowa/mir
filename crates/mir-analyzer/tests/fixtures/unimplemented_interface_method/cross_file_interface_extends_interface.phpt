===file:Readable.php===
<?php
interface Readable {
    public function read(): string;
}
===file:ReadWritable.php===
<?php
interface ReadWritable extends Readable {
    public function write(string $data): void;
}
===file:Stream.php===
<?php
class Stream implements ReadWritable {
    public function write(string $data): void { var_dump($data); }
    # read() inherited from Readable is NOT implemented
}
===expect===
Stream.php: UnimplementedInterfaceMethod: Class Stream must implement Readable::read() from interface
