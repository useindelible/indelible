package app.indelible.auth.viewmodel

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class RegisterStateTest {
    @Test
    fun validPasswordPasses() {
        val error = RegisterState.validatePassword("password123")
        assertNull(error)
    }

    @Test
    fun emptyPasswordFails() {
        val error = RegisterState.validatePassword("")
        assertNotNull(error)
        assertEquals("Password is required", error)
    }

    @Test
    fun shortPasswordFails() {
        val error = RegisterState.validatePassword("short")
        assertNotNull(error)
        assertEquals("Password must be at least 8 characters", error)
    }

    @Test
    fun sevenCharPasswordFails() {
        val error = RegisterState.validatePassword("1234567")
        assertNotNull(error)
    }

    @Test
    fun eightCharPasswordPasses() {
        val error = RegisterState.validatePassword("12345678")
        assertNull(error)
    }

    @Test
    fun matchingPasswordsPasses() {
        val error = RegisterState.validateConfirmPassword("password123", "password123")
        assertNull(error)
    }

    @Test
    fun emptyConfirmPasswordFails() {
        val error = RegisterState.validateConfirmPassword("password123", "")
        assertNotNull(error)
        assertEquals("Please confirm your password", error)
    }

    @Test
    fun mismatchedPasswordsFails() {
        val error = RegisterState.validateConfirmPassword("password123", "different")
        assertNotNull(error)
        assertEquals("Passwords do not match", error)
    }

    @Test
    fun validateReturnsAllErrors() {
        val state = RegisterState()
        val validated = state.validate()
        assertNotNull(validated.displayNameError)
        assertNotNull(validated.emailError)
        assertNotNull(validated.passwordError)
        assertNotNull(validated.confirmPasswordError)
    }

    @Test
    fun validatePassesWithValidInput() {
        val state =
            RegisterState(
                displayName = "Test User",
                email = "user@example.com",
                password = "password123",
                confirmPassword = "password123",
            )
        val validated = state.validate()
        assertNull(validated.displayNameError)
        assertNull(validated.emailError)
        assertNull(validated.passwordError)
        assertNull(validated.confirmPasswordError)
    }

    @Test
    fun isValidReturnsTrueForValidInput() {
        val state =
            RegisterState(
                displayName = "Test User",
                email = "user@example.com",
                password = "password123",
                confirmPassword = "password123",
            )
        assertTrue(state.isValid)
    }

    @Test
    fun isValidReturnsFalseForEmptyName() {
        val state =
            RegisterState(
                displayName = "",
                email = "user@example.com",
                password = "password123",
                confirmPassword = "password123",
            )
        assertFalse(state.isValid)
    }

    @Test
    fun isValidReturnsFalseForMismatchedPasswords() {
        val state =
            RegisterState(
                displayName = "Test User",
                email = "user@example.com",
                password = "password123",
                confirmPassword = "different",
            )
        assertFalse(state.isValid)
    }

    @Test
    fun validateClearsServerError() {
        val state =
            RegisterState(
                displayName = "Test User",
                email = "user@example.com",
                password = "password123",
                confirmPassword = "password123",
                serverError = "some error",
            )
        val validated = state.validate()
        assertNull(validated.serverError)
    }

    @Test
    fun validateDisplayNameRequired() {
        val state =
            RegisterState(
                displayName = "  ",
                email = "user@example.com",
                password = "password123",
                confirmPassword = "password123",
            )
        val validated = state.validate()
        assertNotNull(validated.displayNameError)
        assertEquals("Display name is required", validated.displayNameError)
    }
}
