package app.indelible.feed.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.feed.model.FeedSubscription
import app.indelible.feed.model.UpdateSubscriptionRequest
import app.indelible.feed.repository.FeedRepository
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

sealed class FeedManagementUiState {
    data object Loading : FeedManagementUiState()

    data class Success(
        val subscriptions: List<FeedSubscription>,
    ) : FeedManagementUiState()

    data class Error(
        val message: String,
    ) : FeedManagementUiState()
}

sealed class FeedManagementEffect {
    data class ShowSnackbar(
        val message: String,
    ) : FeedManagementEffect()
}

class FeedManagementViewModel(
    private val repository: FeedRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow<FeedManagementUiState>(FeedManagementUiState.Loading)
    val uiState: StateFlow<FeedManagementUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<FeedManagementEffect>()
    val effects: SharedFlow<FeedManagementEffect> = _effects.asSharedFlow()

    fun loadSubscriptions() {
        _uiState.value = FeedManagementUiState.Loading
        viewModelScope.launch {
            repository
                .listSubscriptions(cursor = null, limit = SUBSCRIPTION_PAGE_SIZE)
                .onSuccess { paginated ->
                    _uiState.value = FeedManagementUiState.Success(paginated.data)
                }.onFailure { error ->
                    _uiState.value =
                        FeedManagementUiState.Error(
                            error.message ?: "Failed to load subscriptions",
                        )
                }
        }
    }

    fun deleteSubscription(subscriptionId: String) {
        val current = _uiState.value as? FeedManagementUiState.Success ?: return
        val removed = current.subscriptions.find { it.id == subscriptionId } ?: return
        val removedIndex = current.subscriptions.indexOf(removed)
        _uiState.value = current.copy(subscriptions = current.subscriptions - removed)

        viewModelScope.launch {
            repository.unsubscribe(subscriptionId).onFailure { error ->
                val restored =
                    (_uiState.value as? FeedManagementUiState.Success)
                        ?.subscriptions
                        ?.toMutableList()
                        ?: mutableListOf()
                restored.add(removedIndex.coerceAtMost(restored.size), removed)
                _uiState.value = (_uiState.value as? FeedManagementUiState.Success)
                    ?.copy(subscriptions = restored)
                    ?: FeedManagementUiState.Success(restored)
                _effects.emit(
                    FeedManagementEffect.ShowSnackbar(error.message ?: "Failed to delete"),
                )
            }
        }
    }

    fun toggleStatus(subscription: FeedSubscription) {
        val newStatus = if (subscription.status == "active") "paused" else "active"
        updateSubscription(subscription.id, UpdateSubscriptionRequest(status = newStatus))
    }

    fun toggleAutoSave(subscription: FeedSubscription) {
        updateSubscription(subscription.id, UpdateSubscriptionRequest(autoSave = !subscription.autoSave))
    }

    fun updateSubscription(
        subscriptionId: String,
        request: UpdateSubscriptionRequest,
    ) {
        val current = _uiState.value as? FeedManagementUiState.Success ?: return
        val original = current.subscriptions.find { it.id == subscriptionId } ?: return
        val originalIndex = current.subscriptions.indexOf(original)

        val optimistic =
            original.copy(
                titleOverride = request.title,
                autoSave = request.autoSave ?: original.autoSave,
                status = request.status ?: original.status,
            )
        val optimisticList =
            current.subscriptions.toMutableList().also {
                it[originalIndex] = optimistic
            }
        _uiState.value = current.copy(subscriptions = optimisticList)

        viewModelScope.launch {
            repository
                .updateSubscription(subscriptionId, request)
                .onSuccess { updated ->
                    val s = _uiState.value as? FeedManagementUiState.Success ?: return@onSuccess
                    val idx = s.subscriptions.indexOfFirst { it.id == subscriptionId }
                    if (idx >= 0) {
                        val finalList = s.subscriptions.toMutableList().also { it[idx] = updated }
                        _uiState.value = s.copy(subscriptions = finalList)
                    }
                }.onFailure { error ->
                    val s = _uiState.value as? FeedManagementUiState.Success
                    if (s != null) {
                        val idx = s.subscriptions.indexOfFirst { it.id == subscriptionId }
                        if (idx >= 0) {
                            val rolledBack =
                                s.subscriptions.toMutableList().also {
                                    it[idx] = original
                                }
                            _uiState.value = s.copy(subscriptions = rolledBack)
                        }
                    }
                    _effects.emit(
                        FeedManagementEffect.ShowSnackbar(error.message ?: "Failed to update"),
                    )
                }
        }
    }

    companion object {
        private const val SUBSCRIPTION_PAGE_SIZE = 100
    }
}
