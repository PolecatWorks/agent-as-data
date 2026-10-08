import { RouterTestingModule } from '@angular/router/testing';
import { HttpClientTestingModule } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TraitsRegistryLayoutComponent } from './traits-registry-layout.component';

describe('TraitsRegistryLayoutComponent', () => {
  let component: TraitsRegistryLayoutComponent;
  let fixture: ComponentFixture<TraitsRegistryLayoutComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TraitsRegistryLayoutComponent, HttpClientTestingModule, RouterTestingModule]
    })
    .compileComponents();

    fixture = TestBed.createComponent(TraitsRegistryLayoutComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
