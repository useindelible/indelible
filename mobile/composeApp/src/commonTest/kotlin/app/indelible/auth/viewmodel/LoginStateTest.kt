package app.indelible.auth.viewmodel

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class LoginStateTest {
    @Test
    fun validEmailPasses() {
        val error = LoginState.validateEmail("user@example.com")
        assertNull(error)
    }

    @Test
    fun emptyEmailFails() {
        val error = LoginState.validateEmail("")
        assertNotNull(error)
        assertEquals("Email is required", error)
    }

    @Test
    fun blankEmailFails() {
        val error = LoginState.validateEmail("   ")
        assertNotNull(error)
        assertEquals("Email is required", error)
    }

    @Test
    fun invalidEmailFormatFails() {
        val error = LoginState.validateEmail("notanemail")
        assertNotNull(error)
        assertEquals("Invalid email format", error)
    }

    @Test
    fun emailMissingDomainFails() {
        val error = LoginState.validateEmail("user@")
        assertNotNull(error)
        assertEquals("Invalid email format", error)
    }

    @Test
    fun emailMissingTldFails() {
        val error = LoginState.validateEmail("user@example")
        assertNotNull(error)
        assertEquals("Invalid email format", error)
    }

    @Test
    fun validateReturnsEmailError() {
        val state = LoginState(email = "bad", password = "validpassword")
        val validated = state.validate()
        assertNotNull(validated.emailError)
        assertNull(validated.passwordError)
    }

    @Test
    fun validateReturnsPasswordError() {
        val state = LoginState(email = "user@example.com", password = "")
        val validated = state.validate()
        assertNull(validated.emailError)
        assertNotNull(validated.passwordError)
        assertEquals("Password is required", validated.passwordError)
    }

    @Test
    fun validateReturnsBothErrors() {
        val state = LoginState(email = "", password = "")
        val validated = state.validate()
        assertNotNull(validated.emailError)
        assertNotNull(validated.passwordError)
    }

    @Test
    fun isValidReturnsTrueForValidInput() {
        val state = LoginState(email = "user@example.com", password = "password123")
        assertTrue(state.isValid)
    }

    @Test
    fun isValidReturnsFalseForInvalidEmail() {
        val state = LoginState(email = "bad", password = "password123")
        assertFalse(state.isValid)
    }

    @Test
    fun isValidReturnsFalseForEmptyPassword() {
        val state = LoginState(email = "user@example.com", password = "")
        assertFalse(state.isValid)
    }

    @Test
    fun validateClearsServerError() {
        val state =
            LoginState(
                email = "user@example.com",
                password = "password123",
                serverError = "some server error",
            )
        val validated = state.validate()
        assertNull(validated.serverError)
    }
}
