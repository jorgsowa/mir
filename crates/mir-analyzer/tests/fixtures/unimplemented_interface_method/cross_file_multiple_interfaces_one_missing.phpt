===file:Serializable.php===
<?php
interface Serializable {
    public function serialize(): string;
}
===file:Identifiable.php===
<?php
interface Identifiable {
    public function getId(): int;
}
===file:Entity.php===
<?php
class Entity implements Serializable, Identifiable {
    public function serialize(): string { return ""; }
    # getId() is NOT implemented
}
===expect===
Entity.php: UnimplementedInterfaceMethod: Class Entity must implement Identifiable::getId() from interface
