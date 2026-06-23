# UIC DOSIPAS Barcode Mobile Library

This library implements the mobile device portion of a UIC DOSIPAS barcoded ticket.
Thas it, it provides routines for resigning the barcode periodically and updating the Level 2 Dynamic Data with the current time, location, etc.

## Android

### Example Usage

```kotlin
import org.uic.dosipas.Dosipas
import org.uic.dosipas.DynamicContentData
import org.uic.dosipas.AndroidKeychain
import org.uic.dosipas.PrivateKey

// Instantiate a new Keychain for generating and storing device keys. This is handle to the operating system's Keychain.
val kc = AndroidKeychain()

// Generate a new device key pair, using the ECDSA P-256 curve.
val privateKey = PrivateKey.genP256(kc)
// Get the private key's ID. This can be used to later retrieve a new handle to the same private key.
privateKey.keyId()
// Retrieve a private key by the key's ID.
val privateKey = PrivateKey.fromKeychain(kc, privateKey.keyId())

// The private key can be turned into the public key for sending to the ticket issuing server.
val publicKey = privateKey.publicKey()
// Retrieve the encoded public key in SEC1 form.
publicKey.publicBytes()
// Get the public key algorithm OID.
publicKey.keyAlgStr() // -> "1.2.840.10045.3.1.7"
// Get the public signature algorithm OID.
publicKey.signingAlgStr() // -> "1.2.840.10045.4.3.2"

// Parse a DOSIPAS ticket returned by the ticketing server
val barcode =  Dosipas.newFromBytes(ticketData)

// Create the Dynamic Content Data record
val dcd = DynamicContentData()
// Set the mobile app ID
dcd.setMobileAppId("ExampleApp")
// Set the generation time
dcd.setTimestamp(Instant.now())
// Set the generation coordinate, usually the Location would be returned by GPS services.
val location = Location("TEST")
location.latitude = 49.33
location.longitude = 6.99
location.accuracy = 10_000.0f
dcd.setLocation(location)
// Put the DCD record in the Level 2 data of the barcode
barcode.setDynamicContent(dcd)

// Sign the barcode, using the private key
barcode.sign(privateKey)
```

## Swift

To use this library in a Swift project, add the repository `https://github.com/TheEnbyperor/uic-dosipas-lib` to your Swift Package Manager.

### Example Usage

```swift
// Instantiate a new Keychain for generating and storing device keys. This is handle to the operating system's Keychain.
let kc = UicDosipas.AppleKeychain()

// Generate a new device key pair, using the ECDSA P-256 curve.
let privateKey = UicDosipas.PrivateKey.genP256(keychain: kc)
// Get the private key's ID. This can be used to later retrieve a new handle to the same private key.
privateKey.keyId()
// Retrieve a private key by the key's ID.
let privateKey = try! UicDosipas.PrivateKey.fromKeychain(keychain: kc, keyId: privateKey.keyId())

// The private key can be turned into the public key for sending to the ticket issuing server.
let publicKey = privateKey.publicKey()
// Retrieve the encoded public key in SEC1 form.
publicKey.publicBytes()
// Get the public key algorithm OID.
publicKey.keyAlgStr() // -> "1.2.840.10045.3.1.7"
// Get the public signature algorithm OID.
publicKey.signingAlgStr() // -> "1.2.840.10045.4.3.2"

// Parse a DOSIPAS ticket returned by the ticketing server
let barcode = try! UicDosipas.Dosipas.newFromBytes(data: barcodeData)

// Create the Dynamic Content Data record
let dcd = UicDosipas.DynamicContentData()
// Set the mobile app ID
try! dcd.setMobileAppId(appId: "ExampleApp")
// Set the generation time
dcd.setTimestamp(time: Date.now)
// Set the generation coordinate, usually the CLLocation would be returned by CoreLocation services.
dcd.setCoordinate(CLLocation(
    coordinate: CLLocationCoordinate2D(latitude: 49.33, longitude: 6.99),
    altitude: 0,
    horizontalAccuracy: 10_000.0,
    verticalAccuracy: 0,
    timestamp: Date.now
))
// A coordinate can also be constructed from just a 2D coordinate.
let position = UicDosipas.GeoCoordinate(CLLocationCoordinate2D(latitude: 49.33, longitude: 6.99))
position.setAccuracyFromMeters(accuracy: 10_000.0)
position.setCoordinateSystem(system: .wgs84)
// Put the DCD record in the Level 2 data of the barcode
try! barcode.setDynamicContent(data: dcd)

// Sign the barcode, using the private key
try! barcode.sign(key: privateKey)
```