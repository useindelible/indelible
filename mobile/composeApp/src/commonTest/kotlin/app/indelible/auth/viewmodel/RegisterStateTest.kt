package app.indelible.auth.viewmodel

import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_confirm_password_required
import indelible.composeapp.generated.resources.auth_display_name_required
import indelible.composeapp.generated.resources.auth_password_min_length
import indelible.composeapp.generated.resources.auth_password_required
import indelible.composeapp.generated.resources.auth_passwords_mismatch
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
        assertEquals(UiMessage(Res.string.auth_password_required), error)
    }

    @Test
    fun shortPasswordFails() {
        val error = RegisterState.validatePassword("short")
        assertNotNull(error)
        assertEquals(UiMessage(Res.string.auth_password_min_length, listOf(8)), error)
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
        assertEquals(UiMessage(Res.string.auth_confirm_password_required), error)
    }

    @Test
    fun mismatchedPasswordsFails() {
        val error = RegisterState.validateConfirmPassword("password123", "different")
        assertNotNull(error)
        assertEquals(UiMessage(Res.string.auth_passwords_mismatch), error)
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
                serverError = UiMessage(Res.string.auth_password_required),
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
        assertEquals(UiMessage(Res.string.auth_display_name_required), validated.displayNameError)
    }
}
