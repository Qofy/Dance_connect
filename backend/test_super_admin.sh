#!/bin/bash


# Test 1: Create a super_admin with explicit flag
echo "=== Test 1: Create super_admin with --add-super-admin flags ==="
./mitote_v026_bike_connect_backend --add-super-admin-email "test-super@example.com" --add-super-admin-password "TestPass123!" 2>&1 | grep -E "(super_admin|admin added|user exists)"

# Test 2: Try to create duplicate (should fail)
echo ""
echo "=== Test 2: Try duplicate (should report 'user exists') ==="
./mitote_v026_bike_connect_backend --add-super-admin-email "test-super@example.com" --add-super-admin-password "TestPass123!" 2>&1 | grep -E "(user exists|super_admin)"

# Test 3: Create a regular admin (first user scenario would make this super_admin if DB is empty)
echo ""
echo "=== Test 3: Create admin with --add-admin flags ==="
./mitote_v026_bike_connect_backend --add-admin-email "test-regular@example.com" --add-admin-password "TestPass123!" 2>&1 | grep -E "(admin added|super_admin)"

echo ""
echo "=== All tests completed ==="



./triggquote8061_backend_development_server_ubuntu_24__aarch64 user add-admin --email="jesusalc@intuivo.com"   --password="emt2dtj_tme7uxb7CBA"
./triggquote8061_backend_development_server_ubuntu_24__aarch64 user add-super-admin --email="jesusalc@intuivo.com"   --password="emt2dtj_tme7uxb7CBA"
./triggquote8061_backend_development_server_ubuntu_24__aarch64 --add-super-admin-email jesusalc@intuivo.com --add-super-admin-password  emt2dtj_tme7uxb7C
./triggquote8061_backend_development_server_ubuntu_24__aarch64 --add-admin-email jesusalc@intuivo.com --add-admin-password  emt2dtj_tme7uxb7C


o /profile and look at the response JSON - check if
feature_visibility shows timeline_organizer: true or if it's missing/null
 if not user this route to enable

// Run this in your browser's DevTools console:
fetch('/profile/enable-all-features', {
method: 'POST',
headers: {
'Content-Type': 'application/json'
}
})
.then(r => r.json())
.then(data => {
console.log('Features enabled!', data);
// Refresh the page to see the menu
location.reload();
})
.catch(e => console.error('Error:', e));



Difference Between admin and super_admin

According to the code, there is currently NO functional difference between admin and
super_admin. Here's what I found:

In the Plans Handler (lines 9-21):

fn require_super_admin(req: &HttpRequest) -> Result<Claims, HttpResponse> {
match req.extensions().get::<Claims>().cloned() {
Some(claims)
if claims
.roles
.iter()
.any(|r| r == "super_admin" || r == "admin") =>  // ← BOTH have same
access
{
Ok(claims)
}
_ => Err(HttpResponse::Forbidden().json(ApiError::forbidden())),
}
}

Both admin AND super_admin are treated identically - they both grant access to
protected routes like the Plan Manager.

The Real Difference is the subscription_tier:

The only meaningful difference is that super_admin gets assigned subscription_tier:
Some("enterprise".into()) while a regular admin doesn't:

- super_admin: Has roles = ["super_admin"] AND subscription_tier = "enterprise"
- admin: Has roles = ["admin"] but subscription_tier = None

This explains the original issue you reported: super_admin users created via AddAdmin
command weren't assigned the enterprise subscription tier, so they saw "Subscriber
Pro" in the profile and "No Plans Available" in billing (because the system checks
subscription_tier, not the role).


My Implementation:

- First user created via --add-admin-email: Automatically gets super_admin role +
enterprise tier (initial setup optimization)
- Any user created via --add-super-admin-email: Explicitly gets super_admin role +
enterprise tier

So the distinction is primarily about subscription/billing access, not about
functional permissions. Both roles currently have admin-level access to all protected
endpoints.

Would you like me to revert my changes, or keep this implementation?
